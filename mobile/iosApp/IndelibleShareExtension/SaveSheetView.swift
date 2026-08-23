import Combine
import SwiftUI

enum ShareSheetState {
    case idle
    case saving
    case saved
    case alreadySaved
    case queued
    case authRequired
    case invalidURL
    case error
}

final class ShareSheetModel: ObservableObject {
    @Published var state: ShareSheetState = .idle
}

struct SaveSheetView: View {
    let url: String
    @ObservedObject var model: ShareSheetModel
    let onSave: () -> Void
    let onCancel: () -> Void
    let onSignIn: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            Spacer()
            VStack(alignment: .leading, spacing: 16) {
                HStack(spacing: 8) {
                    Image(systemName: "bookmark.fill")
                        .foregroundColor(.accentColor)
                    Text("share_title")
                        .font(.headline)
                }

                Divider()

                Text(url)
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .lineLimit(2)

                Text("share_inbox_default")
                    .font(.caption)
                    .foregroundColor(.secondary)

                switch model.state {
                case .saving:
                    HStack {
                        ProgressView()
                        Text("share_saving")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                case .idle:
                    HStack(spacing: 12) {
                        Button(action: onCancel) {
                            Text("common_cancel")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)

                        Button(action: {
                            model.state = .saving
                            onSave()
                        }) {
                            Text("common_save")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.borderedProminent)
                    }
                case .saved:
                    StatusRow(key: "share_saved", systemImage: "checkmark.circle.fill", color: .accentColor)
                case .alreadySaved:
                    StatusRow(key: "share_already_saved", systemImage: "checkmark.circle.fill", color: .accentColor)
                case .queued:
                    StatusRow(key: "share_offline", systemImage: "wifi.slash", color: .secondary)
                case .authRequired:
                    StatusRow(key: "share_sign_in", systemImage: "person.crop.circle.badge.exclamationmark", color: .red)
                    Button(action: onSignIn) {
                        Text("share_open_indelible")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.borderedProminent)
                case .invalidURL:
                    StatusRow(key: "share_invalid_url", systemImage: "exclamationmark.triangle.fill", color: .red)
                    Button("common_cancel", action: onCancel)
                        .buttonStyle(.bordered)
                case .error:
                    StatusRow(key: "share_error", systemImage: "exclamationmark.triangle.fill", color: .red)
                    Button("common_cancel", action: onCancel)
                        .buttonStyle(.bordered)
                }
            }
            .padding(24)
            .background(Color(UIColor.systemBackground))
            .cornerRadius(16)
            .padding(.horizontal, 16)
            .padding(.bottom, 8)
        }
        .background(Color.black.opacity(0.3))
        .ignoresSafeArea()
    }
}

private struct StatusRow: View {
    let key: LocalizedStringKey
    let systemImage: String
    let color: Color

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: systemImage)
                .foregroundColor(color)
            Text(key)
                .font(.subheadline)
        }
    }
}
