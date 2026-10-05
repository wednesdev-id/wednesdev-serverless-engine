package main

import (
	bindings "wednes-sdk-go/bindings"
)

func init() {
	bindings.SetFunction(impl{})
}

type impl struct{}

func (i impl) Handle(req bindings.FunctionRequest) bindings.FunctionResponse {
	return bindings.FunctionResponse{
		Status: 200,
		Body:   []byte("Hello from TinyGo WASM!"),
	}
}

func main() {}
