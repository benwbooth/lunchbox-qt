#!/usr/bin/env python3
"""Extract a value-free schema census from a LaunchBox installation.

The output intentionally excludes filenames, element text, attribute values,
account data, paths stored inside XML, and license material.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from pathlib import Path
import xml.etree.ElementTree as ET


def document_group(data_root: Path, path: Path) -> str:
    relative = path.relative_to(data_root)
    if len(relative.parts) == 1:
        return relative.name
    return f"{relative.parts[0]}/*.xml"


def inspect_xml(path: Path) -> dict[str, object]:
    root = ET.parse(path).getroot()
    records: Counter[str] = Counter()
    record_fields: dict[str, set[str]] = defaultdict(set)
    record_attributes: dict[str, set[str]] = defaultdict(set)
    for record in root:
        records[record.tag] += 1
        record_attributes[record.tag].update(record.attrib)
        record_fields[record.tag].update(child.tag for child in record)
    return {
        "root": root.tag,
        "records": records,
        "record_fields": record_fields,
        "record_attributes": record_attributes,
    }


def launchbox_version(install_root: Path) -> str | None:
    deps = install_root / "Core" / "LaunchBox.deps.json"
    if not deps.is_file():
        return None
    payload = json.loads(deps.read_text(encoding="utf-8"))
    for target in payload.get("targets", {}).values():
        for package in target:
            if package.startswith("LaunchBox/"):
                return package.split("/", 1)[1]
    return None


def sorted_directory_names(root: Path) -> list[str]:
    if not root.is_dir():
        return []
    return sorted(entry.name for entry in root.iterdir() if entry.is_dir())


def combined_rom_census(data_root: Path) -> dict[str, int]:
    """Count combined-version shapes without retaining any library values."""
    counts: Counter[str] = Counter()
    for path in sorted((data_root / "Platforms").glob("*.xml")):
        try:
            root = ET.parse(path).getroot()
        except (ET.ParseError, OSError):
            continue
        games = {
            game.findtext("ID", default=""): game.findtext(
                "ApplicationPath", default=""
            )
            for game in root.findall("Game")
        }
        grouped: dict[str, list[ET.Element]] = defaultdict(list)
        for application in root.findall("AdditionalApplication"):
            name = application.findtext("Name", default="")
            if name.startswith("Play ") and name.endswith(" Version..."):
                grouped[application.findtext("GameID", default="")].append(
                    application
                )
        for game_id, applications in grouped.items():
            if len(applications) < 2 or game_id not in games:
                continue
            counts["multi_entry_groups"] += 1
            counts["version_application_records"] += len(applications)
            if any(
                application.findtext("ApplicationPath", default="")
                == games[game_id]
                for application in applications
            ):
                counts["groups_with_primary_path_record"] += 1
            if all(
                application.findtext("Name", default="")
                == "Play "
                + application.findtext("Version", default="")
                + " Version..."
                for application in applications
            ):
                counts["groups_using_exact_version_name_formula"] += 1
            priorities = sorted(
                int(application.findtext("Priority", default="0"))
                for application in applications
            )
            if priorities == list(range(1, len(applications) + 1)):
                counts["groups_with_contiguous_one_based_priorities"] += 1
    return dict(sorted(counts.items()))


def additional_application_census(
    data_root: Path,
) -> dict[str, int | None]:
    """Count editor-relevant shapes without retaining IDs, paths, or names."""
    counts: Counter[str] = Counter()
    application_ids: set[str] = set()
    referenced_application_ids: list[str] = []
    priorities: list[int] = []
    for path in sorted((data_root / "Platforms").glob("*.xml")):
        try:
            root = ET.parse(path).getroot()
        except (ET.ParseError, OSError):
            continue
        for application in root.findall("AdditionalApplication"):
            counts["records"] += 1
            application_id = application.findtext("Id", default="").strip()
            if application_id:
                application_ids.add(application_id)
            if not application.findtext("ApplicationPath", default="").strip():
                counts["records_with_empty_application_path"] += 1
            try:
                priorities.append(
                    int(application.findtext("Priority", default="0"))
                )
            except ValueError:
                counts["records_with_invalid_priority"] += 1
            use_emulator = (
                application.findtext("UseEmulator", default="").lower()
                == "true"
            )
            emulator_id = application.findtext(
                "EmulatorId", default=""
            ).strip()
            if use_emulator and not emulator_id:
                counts["emulated_records_without_emulator_id"] += 1
            if not use_emulator and emulator_id:
                counts["direct_records_with_emulator_id"] += 1
            if (
                application.findtext("UseDosBox", default="").lower()
                == "true"
            ):
                counts["dosbox_records"] += 1
            if (
                application.findtext("AutoRunBefore", default="").lower()
                == "true"
            ):
                counts["auto_run_before_records"] += 1
            if (
                application.findtext("AutoRunAfter", default="").lower()
                == "true"
            ):
                counts["auto_run_after_records"] += 1
            if (
                application.findtext("WaitForExit", default="").lower()
                == "true"
            ):
                counts["wait_for_exit_records"] += 1
        for game_save in root.findall("GameSave"):
            application_id = game_save.findtext(
                "AdditionalApplicationId", default=""
            ).strip()
            if application_id:
                referenced_application_ids.append(application_id)

    counts["game_save_references"] = len(referenced_application_ids)
    counts["resolved_game_save_references"] = sum(
        application_id in application_ids
        for application_id in referenced_application_ids
    )
    count_names = (
        "records",
        "records_with_empty_application_path",
        "records_with_invalid_priority",
        "emulated_records_without_emulator_id",
        "direct_records_with_emulator_id",
        "dosbox_records",
        "auto_run_before_records",
        "auto_run_after_records",
        "wait_for_exit_records",
        "game_save_references",
        "resolved_game_save_references",
    )
    result: dict[str, int | None] = {
        name: counts[name] for name in sorted(count_names)
    }
    result["minimum_priority"] = min(priorities, default=None)
    result["maximum_priority"] = max(priorities, default=None)
    return result


def build_census(install_root: Path) -> dict[str, object]:
    data_root = install_root / "Data"
    if not data_root.is_dir():
        raise SystemExit(f"LaunchBox Data directory not found below {install_root}")

    grouped: dict[str, dict[str, object]] = {}
    for path in sorted(data_root.rglob("*.xml")):
        group = document_group(data_root, path)
        aggregate = grouped.setdefault(
            group,
            {
                "file_count": 0,
                "total_bytes": 0,
                "root_elements": Counter(),
                "record_counts": Counter(),
                "record_fields": defaultdict(set),
                "record_attributes": defaultdict(set),
                "parse_error_count": 0,
            },
        )
        aggregate["file_count"] += 1
        aggregate["total_bytes"] += path.stat().st_size
        try:
            inspected = inspect_xml(path)
        except (ET.ParseError, OSError):
            aggregate["parse_error_count"] += 1
            continue
        aggregate["root_elements"][inspected["root"]] += 1
        aggregate["record_counts"].update(inspected["records"])
        for record, fields in inspected["record_fields"].items():
            aggregate["record_fields"][record].update(fields)
        for record, attributes in inspected["record_attributes"].items():
            aggregate["record_attributes"][record].update(attributes)

    serializable_groups: dict[str, object] = {}
    for group, aggregate in sorted(grouped.items()):
        serializable_groups[group] = {
            "file_count": aggregate["file_count"],
            "total_bytes": aggregate["total_bytes"],
            "root_elements": dict(sorted(aggregate["root_elements"].items())),
            "record_counts": dict(sorted(aggregate["record_counts"].items())),
            "record_fields": {
                record: sorted(fields)
                for record, fields in sorted(aggregate["record_fields"].items())
            },
            "record_attributes": {
                record: sorted(attributes)
                for record, attributes in sorted(
                    aggregate["record_attributes"].items()
                )
                if attributes
            },
            "parse_error_count": aggregate["parse_error_count"],
        }

    return {
        "privacy": (
            "Value-free schema census: no source filenames, element text, "
            "attribute values, stored paths, account data, or license data."
        ),
        "launchbox_version": launchbox_version(install_root),
        "install_layout_directories": sorted_directory_names(install_root),
        "data_layout_directories": sorted_directory_names(data_root),
        "additional_applications": additional_application_census(data_root),
        "combined_roms": combined_rom_census(data_root),
        "document_groups": serializable_groups,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("install_root", type=Path)
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("analysis/real-install-schema.json"),
    )
    args = parser.parse_args()

    census = build_census(args.install_root)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(census, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {args.output}")


if __name__ == "__main__":
    main()
