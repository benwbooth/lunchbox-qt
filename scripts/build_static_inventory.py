#!/usr/bin/env python3
"""Build a reproducible structural inventory from the installed/decompiled oracle."""

from __future__ import annotations

import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CORE = ROOT / "oracle" / "LaunchBox" / "Core"
DECOMPILED = ROOT / "decompiled"
THEMES = ROOT / "oracle" / "LaunchBox" / "Themes"
OUTPUT = ROOT / "analysis" / "static-inventory.json"

ASSEMBLIES = [
    "LaunchBox.dll",
    "BigBox.dll",
    "Unbroken.dll",
    "Unbroken.LaunchBox.dll",
    "Unbroken.LaunchBox.LocalDb.dll",
    "Unbroken.LaunchBox.Plugins.dll",
    "Unbroken.LaunchBox.SourceGenerators.dll",
    "Unbroken.LaunchBox.Windows.dll",
    "Unbroken.LaunchBox.Windows.BigPEmu.dll",
    "Unbroken.LaunchBox.Windows.Dolphin.dll",
    "Unbroken.LaunchBox.Windows.Mame.dll",
    "Unbroken.LaunchBox.Windows.Pcsx2.dll",
    "Unbroken.LaunchBox.Windows.PlaylistProvider.dll",
    "Unbroken.LaunchBox.Windows.RetroArch.dll",
    "Unbroken.LaunchBox.Windows.ScummVm.dll",
    "Unbroken.LaunchBox.Windows.Xemu.dll",
]

FAILURE_MARKERS = (
    "Error while decompiling",
    "DecompilerException",
    "Unable to decompile",
    "Could not resolve type",
)


def files(root: Path, pattern: str) -> list[Path]:
    if not root.exists():
        return []
    return sorted(path for path in root.rglob(pattern) if path.is_file())


def rel_strings(paths: list[Path], root: Path) -> list[str]:
    return [path.relative_to(root).as_posix() for path in paths]


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def source_metrics(root: Path) -> dict[str, int]:
    cs_files = files(root, "*.cs")
    failure_files = 0
    empty_noinline_files = 0
    transformer_named_files = 0
    empty_noinline = re.compile(
        r"\[MethodImpl\(MethodImplOptions\.NoInlining\)\]\s+"
        r"public[^\n]*\s*\{\s*\}",
        re.MULTILINE,
    )
    for path in cs_files:
        text = read_text(path)
        failure_files += any(marker in text for marker in FAILURE_MARKERS)
        empty_noinline_files += bool(empty_noinline.search(text))
        transformer_named_files += "Transformer" in path.name
    return {
        "csharp_files": len(cs_files),
        "baml_files": len(files(root, "*.baml")),
        "decompiler_failure_marker_files": failure_files,
        "empty_public_noinline_method_marker_files": empty_noinline_files,
        "transformer_named_files": transformer_named_files,
    }


def stems(root: Path) -> list[str]:
    return sorted(path.stem for path in files(root, "*.cs"))


def main() -> None:
    if not CORE.exists() or not DECOMPILED.exists():
        raise SystemExit("Install and decompile the oracle before building inventory")

    assembly_inventory = []
    for assembly in ASSEMBLIES:
        binary = CORE / assembly
        source = DECOMPILED / binary.stem
        assembly_inventory.append(
            {
                "assembly": assembly,
                "installed_bytes": binary.stat().st_size,
                **source_metrics(source),
            }
        )

    desktop_root = (
        DECOMPILED
        / "LaunchBox"
        / "Unbroken"
        / "LaunchBox"
        / "Windows"
        / "Desktop"
    )
    bigbox_root = (
        DECOMPILED
        / "BigBox"
        / "Unbroken"
        / "LaunchBox"
        / "Windows"
        / "BigBox"
    )
    plugin_root = (
        DECOMPILED
        / "Unbroken.LaunchBox.Plugins"
        / "Unbroken"
        / "LaunchBox"
        / "Plugins"
    )

    options_source = read_text(bigbox_root / "Data" / "OptionsPages.cs")
    bigbox_option_pages = sorted(
        set(re.findall(r"public static OptionsPage (Get\w+)\(", options_source))
    )

    inventory = {
        "oracle": {
            "product_version": "13.27",
            "installer_sha256": (
                "19deeee55c135ffb1b720bcfcdecdd9e103ac86a6c47ffdc2b6b5a4af83b6481"
            ),
            "target_framework": "net9.0/win-x64",
        },
        "assemblies": assembly_inventory,
        "ui_resources": {
            "launchbox_baml": rel_strings(
                files(DECOMPILED / "LaunchBox", "*.baml"),
                DECOMPILED / "LaunchBox",
            ),
            "bigbox_baml": rel_strings(
                files(DECOMPILED / "BigBox", "*.baml"),
                DECOMPILED / "BigBox",
            ),
            "shared_baml": rel_strings(
                files(DECOMPILED / "Unbroken.LaunchBox.Windows", "*.baml"),
                DECOMPILED / "Unbroken.LaunchBox.Windows",
            ),
            "installed_theme_xaml": rel_strings(
                files(THEMES, "*.xaml"),
                THEMES,
            ),
        },
        "semantic_surfaces": {
            "desktop_menu_actions": stems(desktop_root / "MenuActions"),
            "desktop_view_models": stems(desktop_root / "ViewModels"),
            "bigbox_menu_actions": stems(bigbox_root / "MenuActions"),
            "bigbox_view_models": stems(bigbox_root / "ViewModels"),
            "bigbox_option_pages": bigbox_option_pages,
            "plugin_contracts": rel_strings(files(plugin_root, "*.cs"), plugin_root),
        },
        "limitations": {
            "method_body_protection_observed": True,
            "meaning": (
                "Structural names/resources/contracts are useful; many implementation "
                "bodies are runtime-restored or dispatched and are not faithfully "
                "represented by the static C# output."
            ),
        },
    }

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {OUTPUT}")


if __name__ == "__main__":
    main()
