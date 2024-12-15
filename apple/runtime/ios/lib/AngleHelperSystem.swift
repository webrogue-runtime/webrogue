@objc 
class AngleHelperSystem: NSObject {
    @objc
    public override init() {
        DispatchQueue.main.sync {
            SDL_Init(SDL_INIT_VIDEO);
        }
    }

    @objc
    public func makeWindow() -> AngleHelperWindow {
        DispatchQueue.main.sync {
            AngleHelperWindow()
        }
    }
}
