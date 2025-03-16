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
            run(path: Bundle.main.url(forResource: "aot", withExtension: "wrapp")!.relativePath)
        }

        return result
    }

    @objc
    func runPathNotification(notification: Notification) {
        guard let path = notification.object as? String else { return }
        run(path: path)
    }

    func run(path: String, completion: ((Int) -> Void)? = nil) {
        DispatchQueue.global(qos: .userInteractive).async {
            self.isWebrogueWindowVisible = false
            let ret_code = path.utf8CString.withUnsafeBufferPointer {
                Int(webrogueObjCMain($0.baseAddress!))
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
