// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use wasm_builder_runner::{build_current_project_with_rustflags, WasmBuilderSource};
use std::env;

fn main() {
	let cargo = env::var("CARGO").expect("`CARGO` is always set by cargo.");
	if cargo.ends_with("rls") {
		env::set_var("CARGO", cargo.replace("rls", "cargo"));
	}
	build_current_project_with_rustflags(
		"wasm_binary.rs",
		WasmBuilderSource::Crates("1.0.6"),
		// This instructs LLD to export __heap_base as a global variable, which is used by the
		// external memory allocator.
		"-Clink-arg=--export=__heap_base",
	);
}
