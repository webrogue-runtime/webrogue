public class WebrogueAppDelegate: SDLUIKitDelegate {
    static var shared: WebrogueAppDelegate!
    var webrogueWindow: UIWindow!
    var isWebrogueWindowVisible = true

    override public var window: UIWindow! {
        get {
            isWebrogueWindowVisible ? webrogueWindow : super.window
        }
        set {}
    }

    override public func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey : Any]? = nil
    ) -> Bool {
        WebrogueAppDelegate.shared = self

        let result = super.application(
            application,
            didFinishLaunchingWithOptions: launchOptions
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(runPathNotification),
            name: .init(rawValue: "WebrogueRunPath"),
            object: nil
        )
        if let viewController = webrogueControllerBlock?() {
            webrogueWindow = UIWindow(frame: UIScreen.main.bounds)
            webrogueWindow.rootViewController = viewController
            webrogueWindow.makeKeyAndVisible()
        } else {
            let fileManager = FileManager.default
            let documentDirPath = fileManager.urls(
                for: .documentDirectory,
                in: .userDomainMask
            ).first!.relativePath
            let dataDirPath = documentDirPath + "/.webrogue"
            if !fileManager.fileExists(atPath: dataDirPath) {
                try! fileManager.createDirectory(atPath: dataDirPath, withIntermediateDirectories: true)
            }
            run(
                path: Bundle.main.url(forResource: "aot", withExtension: "wrapp")!.relativePath,
                dataPath: dataDirPath + "/data"
            )
        }

        return result
    }

    @objc
    func runPathNotification(notification: Notification) {
        guard let object = notification.object as? [String] else { return }
        run(path: object[0], dataPath: object[1])
    }

    func run(path: String, dataPath: String, completion: ((Int) -> Void)? = nil) {
        DispatchQueue.global(qos: .userInteractive).async {
            self.isWebrogueWindowVisible = false
            let ret_code = path.utf8CString.withUnsafeBufferPointer { pathBuff in
                dataPath.utf8CString.withUnsafeBufferPointer { dataPathBuff in
                    Int(webrogueObjCMain(
                        pathBuff.baseAddress!,
                        dataPathBuff.baseAddress!
                    ))
                }
            }
            self.isWebrogueWindowVisible = true
            completion?(ret_code)
        }
    }

    public override func application(
        _ app: UIApplication,
        open url: URL,
        options: [UIApplication.OpenURLOptionsKey : Any] = [:]
    ) -> Bool {
        NotificationCenter.default.post(
            name: .init(rawValue: "WebrogueURL"),
            object: url
        )
        return true
    }
}
