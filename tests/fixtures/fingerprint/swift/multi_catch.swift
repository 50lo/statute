enum Decode {
    static func describe(_ data: Data) -> String {
        do {
            _ = try JSONDecoder().decode([String].self, from: data)
            return "ok"
        } catch let DecodingError.typeMismatch(_, ctx), let DecodingError.valueNotFound(_, ctx) {
            return ctx.debugDescription
        } catch {
            return "other"
        }
    }
}
