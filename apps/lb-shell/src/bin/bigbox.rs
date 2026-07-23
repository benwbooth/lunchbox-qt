fn main() {
    lb_shell::run(lb_shell::ShellMode::BigBox);
}

#[cfg(test)]
mod tests {
    #[test]
    fn links_qt_initializers() {
        lb_shell::initialize_qt();
    }
}
