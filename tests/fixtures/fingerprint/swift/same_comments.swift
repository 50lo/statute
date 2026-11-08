/// An invoice.
@MainActor public struct Invoice {
    /* stored value */ var total: Int = 1
    /// Approve the invoice.
    func approve(name: String) -> Int { return total }
    struct Item { func value() -> Int { return 1 } }
}
func global(_ value: Int) -> Int { return value }
protocol P { func run() }
extension P { func run() {} }
extension Invoice { func move(to destination: Int) {} }
