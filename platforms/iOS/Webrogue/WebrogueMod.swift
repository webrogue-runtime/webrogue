class WebrogueMod {
    let name: String;
    let isActive: Bool

    init(name: String, isActive: Bool) {
        self.name = name
        self.isActive = isActive
    }


    func delete() -> Bool {
        guard let modDirs = WebrogueMod.getModDirs() else { return false }
        let dir = isActive ? modDirs.active : modDirs.inactive
        do {
            try FileManager.default.removeItem(atPath: dir + "/" + name)
            return true
        } catch { return false }
    }

    private static func getModDirs() -> (active: String, inactive: String)? {
        guard let dataDirectory = getDataDirectory() else {
            return nil
        }
        let activeModsDirectory = dataDirectory.appending("/mods")
        let inactiveModsDirectory = dataDirectory.appending("/inactive_mods")
        return (activeModsDirectory, inactiveModsDirectory)
    }

    static func getAll() -> ([WebrogueMod], [WebrogueMod]) {
        guard let modDirs = getModDirs() else { return ([], []) }
        do {
            let activeMods = try FileManager.default.contentsOfDirectory(
                atPath: modDirs.active
            ).map {
                getMod(
                    name: $0,
                    path: modDirs.active,
                    isActive: true
                )
            }
            let inactiveMods = try FileManager.default.contentsOfDirectory(
                atPath: modDirs.inactive
            ).map {
                getMod(
                    name: $0,
                    path: modDirs.inactive,
                    isActive: false
                )
            }
            return (activeMods, inactiveMods)
        } catch {
            return ([], [])
        }
    }

    private static func getMod(
        name:String,
        path: String,
        isActive: Bool
    ) -> WebrogueMod {
        WebrogueMod(name: name, isActive: isActive)
    }
}
