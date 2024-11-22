import SwiftUI
import WebrogueCommon

struct ContainerReferenceView: View {
    let ref: ContainerReference
    @StateObject private var viewModel = ContainerReferenceViewModel()
    @State var s = ""


    init(for ref: ContainerReference) {
        self.ref = ref
    }

    var body: some View {
        GeometryReader { _ in
            VStack(alignment: .leading) {
                Group {
                    let status = if viewModel.isRunning {
                        "running"
                    } else {
                        "idle"
                    }
                    Text("Path: \(ref.path)")
                    Text("SHA256: \(ref.metadata.sha256)")
                        .lineLimit(1)
                    Text("Status: \(status)")
                    Button(action: {
                        assert((viewModel.terminate != nil) == viewModel.isRunning)
                        if let terminate = viewModel.terminate {
                            terminate()
                        } else {
                            Task { @MainActor in
                                viewModel.clear()
                                viewModel.isRunning = true
                                await ref.launch(
                                    stdoutHandler: { data in
                                        viewModel.append(data)
                                    },
                                    terminatorSetter: { @MainActor terminate in
                                        viewModel.terminate = terminate
                                    }
                                )
                                viewModel.isRunning = false
                                viewModel.terminate = nil
                            }
                        }
                    }, label: {
                        let label = if viewModel.isRunning {
                            "Terminate"
                        } else {
                            "Launch"
                        }
                        Text(label)
                    })
                        .frame(alignment: .center)
                }
                    .padding(.horizontal, 8)

                GeometryReader { _ in
                    ScrollView {
                        VStack(alignment: .leading, spacing: .zero) {
                            Text(viewModel.decodedData)
                                .font(.system(size: 12, design: .monospaced))
                                .textSelection(.enabled)
                            Color.clear
                        }
                    }
                        .background { Color(nsColor: .controlBackgroundColor) }
                }
            }
        }
    }
}
