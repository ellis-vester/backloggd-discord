package types

type Result[T any] struct {
	Data T
	Err  error
}
