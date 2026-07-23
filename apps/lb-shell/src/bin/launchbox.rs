fn main() {
    lb_shell::run(lb_shell::ShellMode::LaunchBox);
}

#[cfg(test)]
mod tests {
    #[test]
    fn links_qt_initializers() {
        lb_shell::initialize_qt();
    }
}
