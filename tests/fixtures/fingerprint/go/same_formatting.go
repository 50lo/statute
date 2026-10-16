//go:build linux
package sessioncommit
import "strings"
type Publisher[T any] struct { threshold int; items []T }
func (p *Publisher[T]) Decide(evidence string) bool { return strings.Contains(evidence, "git commit") }
func (p Publisher[T]) Size() int {
	return len(p.items)
}
//go:noinline
func Score(a, b int) int { s := "a  b"; return a + b + len(s) }
type ( ID string; Alias = ID )
func init() {}
func init() {}
