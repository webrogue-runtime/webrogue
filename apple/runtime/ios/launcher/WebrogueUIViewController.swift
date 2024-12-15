import SwiftUI

class WebrogueUIViewController: UIViewController {

    let swiftUIView = UIHostingController(rootView: WebrogueView())

    override func viewWillAppear(_ animated: Bool) {
        view.backgroundColor = UIColor.systemBackground
        super.viewWillAppear(animated)
    }

    override func viewDidLoad() {
        let subview = swiftUIView.view!
        view.addSubview(subview)
        subview.translatesAutoresizingMaskIntoConstraints = false

        view.addConstraints([.left, .right, .top, .bottom].map {
            NSLayoutConstraint(
                item: subview,
                attribute: $0,
                relatedBy: NSLayoutConstraint.Relation.equal,
                toItem: view,
                attribute: $0,
                multiplier: 1,
                constant: 0
            )
        })
    }
}
