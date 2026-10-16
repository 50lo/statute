//go:build linux

// Package sessioncommit links Sessions to commits.
package sessioncommit

import "strings"

// Publisher decides which Sessions contributed to a commit.
type Publisher[T any] struct {
	threshold int
	items     []T
}

// Decide reports whether the evidence is enough.
func (p *Publisher[T]) Decide(evidence string) bool {
	return strings.Contains(evidence, "git commit") // the only evidence that counts
}

func (p Publisher[T]) Size() int { return len(p.items) }

//go:noinline
func Score(a, b int) int {
	s := "a  b" // whitespace inside a string is content
	return a + b + len(s)
}

type (
	ID    string
	Alias = ID
)

func init() {}

func init() {}
