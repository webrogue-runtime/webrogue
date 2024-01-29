class WebrogueViewController: UIViewController {
    var animationTime: TimeInterval { 0.3 }
    lazy var tableView = UITableView().then {
        $0.dataSource = self
        $0.delegate = self
        $0.backgroundColor = .clear
        $0.register(WebrogueTableCell.self, forCellReuseIdentifier: "mod")
    }

    var activeMods: [WebrogueMod] = []
    var inactiveMods: [WebrogueMod] = []

    override func viewWillAppear(_ animated: Bool) {
        view.backgroundColor = UIColor(named: "customBackgroundColor")
        super.viewWillAppear(animated)
    }

    override func viewDidLoad() {
        view.addSubview(tableView)
        tableView.snp.makeConstraints {
            $0.edges.equalToSuperview()
        }
        updateMods()
        navigationItem.title = "Mods"
        navigationItem.rightBarButtonItems = [
            UIBarButtonItem(barButtonSystemItem: .play, target: self, action: #selector(play)),
            UIBarButtonItem(barButtonSystemItem: .edit, target: self, action: #selector(edit))
        ]
        navigationItem.leftBarButtonItems = [
            UIBarButtonItem(barButtonSystemItem: .add, target: self, action: #selector(add))
        ]
    }
}

extension WebrogueViewController {
    func updateMods() {
        (activeMods, inactiveMods) = WebrogueMod.getAll()
        tableView.reloadData()
    }

    @objc
    func edit() {
        UIView.animate(withDuration: animationTime) {
            self.tableView.isEditing.toggle()
        }
    }

    @objc
    func play() {
        WebrogueAppDelegate.shared?.runGame() { [weak self] _ in
            self?.updateMods()
        }
    }

    @objc
    func add() {
        // TODO implememt
    }
}

extension WebrogueViewController: UITableViewDataSource {
    func numberOfSections(in tableView: UITableView) -> Int {
        return 2
    }

    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        switch section {
        case 0: return activeMods.count
        case 1: return inactiveMods.count
        default: return 0
        }
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let mod: WebrogueMod
        switch indexPath.section {
        case 0:
            mod = activeMods[indexPath.row]
        case 1:
            mod = inactiveMods[indexPath.row]
        default: return UITableViewCell()
        }
        let cell = tableView.dequeueReusableCell(
            withIdentifier: "mod",
            for: indexPath
        ) as! WebrogueTableCell
        cell.configure(with: mod)
        return cell
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        WebrogueTableHeaderView(isActive: section == 0)
    }
}

extension WebrogueViewController: UITableViewDelegate {
    func tableView(_ tableView: UITableView, canEditRowAt indexPath: IndexPath) -> Bool {
        true
    }
    func tableView(_ tableView: UITableView, commit editingStyle: UITableViewCellEditingStyle, forRowAt indexPath: IndexPath) {
        switch editingStyle {
        case .delete:
            let isActiveMod = indexPath.section == 0
            let mod = (isActiveMod ? activeMods : inactiveMods)[indexPath.row]
            if !mod.delete() {
                break
            }
            if isActiveMod {
                activeMods.remove(at: indexPath.row)
            } else {
                inactiveMods.remove(at: indexPath.row)
            }
            tableView.deleteRows(at: [indexPath], with: .automatic)
        default:
            break
        }
    }
}
