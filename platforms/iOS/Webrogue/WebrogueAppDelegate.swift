@main
public class WebrogueAppDelegate: SDLUIKitDelegate {
    static var shared: WebrogueAppDelegate?
    var myWindow: UIWindow?

    override public func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplicationLaunchOptionsKey : Any]? = nil
    ) -> Bool {
        WebrogueAppDelegate.shared = self

        let result = super.application(
            application,
            didFinishLaunchingWithOptions: launchOptions
        )
        myWindow = UIWindow(frame: UIScreen.main.bounds)
        let viewController = WebrogueViewController()
        let navigationController = UINavigationController(
            rootViewController: viewController
        )
        myWindow?.rootViewController = navigationController
        myWindow?.makeKeyAndVisible()
        // runGame()
        return result
    }


    func runGame(completion: ((Int) -> Void)? = nil) {
        DispatchQueue.main.asyncAfter(deadline: .now()) {
            let ret_code = Int(webrogueMain())
            completion?(ret_code)
        }
    }
}
