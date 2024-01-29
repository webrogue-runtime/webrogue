class WebrogueTableHeaderView: UIView {
    let label = UILabel()
    let isActive: Bool

    init(isActive: Bool) {
        self.isActive = isActive
        super.init(frame: .zero)
        setup()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setup() {
        addSubview(label)
        label.snp.makeConstraints {
            $0.top.leading.bottom.equalToSuperview().inset(8)
        }
        label.text = isActive ? "Active mods:" : "Inactive mods:"
        label.font = .systemFont(ofSize: 12)
    }
}
