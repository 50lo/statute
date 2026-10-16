//go:build linux

// Package sessioncommit links Sessions to commits.
package sessioncommit

import (
	"fmt"
	"strings"
)

// Publisher decides which Sessions contributed to a commit.
type Publisher[T any] struct {
	threshold int
	items     []T
}

// Decide reports whether the evidence is enough.
func (p *Publisher[T]) Decide(evidence string) bool {
	return strings.Contains(evidence, "git commit") // the only evidence that counts
}

func (p Publisher[T]) Size() int { return len(p.items) + 1 }

func (p *Publisher[T]) Reset() { p.items = nil }

func Helper() { fmt.Println("new") }

//go:noinline
func Score(a, b int) int {
	s := "a  b" // whitespace inside a string is content
	return a + b + len(s)
}

type (
	ID    string
	Alias = ID
	Extra int
)

func init() {}

func init() {}
