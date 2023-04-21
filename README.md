# Directory Listing using Streams in Rust

Procedure to create a generic stream iterator that can abstract out
the backends (i.e., whether I'm using Tokio against a real filesystem
or an in-memory implementation I hand-rolled myself, or someone else's
system).

## Concept: Pinning

- Move: All values in Rust can be moved in memory, which means to
copy the contents of the value and make the old value invalid. This is
a combination of pass-by-value semantics and use-at-most-once logic.
- Pinning: If there is a shared reference `&T`, then the memory address
of the `T` value itself probably (I think?) can't be moved. On the
other hand, if under `&mut T`, an *exclusive* reference, the value
itself can be freely moved under any circumstances. Particularly,
`mem::swap(&mut _, &mut _)` can be used to copy the de-referenced
contents of the left and right values, respectively, and assign them
into each other's opposite memory addresses. So, in Rust, pinning is
implemented by *hiding the `&mut T` exclusive references*, and relying
on `AsRef<_>` and `Deref<_>` to carry out memory accesses indirectly.
- `Pin<&mut _>`: Takes an exclusive reference `&mut T` and then hides
it. It implements the appropriate `AsRef` and `Deref` traits to allow
indirect access.
- `Unpin`: Many values in Rust do not *require* pinning to work correctly.
If a type `T` implements `Unpin`, then `Pin<&mut T>` can be destroyed
and reveal the exclusive reference by using `&mut *my_pinned_t`
(given that `my_pinned_t: Pin<&mut T>`). `Unpin` is implemented by
default for all values unless it impelements / should implement `!Unpin`.
- `Box<Pin<_>>`: Is a common combination that represents a value in
heap memory that may not be moved (only destroyed).
- Why Pin: Futures (asynchronous threads) frequently get swapped across
threads while holding a reference to values. As described, Safe Rust
allows any value to be moved while the thread holds an exclusive reference to it.
In asynchronous code, holding an exclusive reference does not guarantee
the object will be accessible at a constant pointer location. To
elaborate, the object will still be *live* due to the lifetime
semantics, but it won't be *exactly there* because it could be at a
difference memory location. So it's hazardous to hold these pointers
because they can be invalidated, even in safe Rust, whether these values
reside in heap memory or stack memory or even read-only memory.

## Concept: Stream

- Kinda like an asynchronous iterator tbh, didn't look too much into it.
- You return `Some(_)`, you still getting values, and you return `None`,
boom, the stream is over.
- You also need to return a correct implementation of `size_hint(_)`,
which gives an absolute upper bound of the number of elements to come.
An always-correct implementation gives positive infinity, represented
by `None` (as opposed to a concrete value, a `Some(usize)`). This
`size_hint(_)` implementation is provided by default.
- You can make a `Stream` behave like an `Iterator` by importing
`tokio_stream::StreamExt`, if you're using the `tokio-stream` crate.

## Concept: Zero-Sized Types

- These things are structures that occupy no space in memory.
- Lots of funky things happen when it comes to addressing and allocating
them, but since ZST's are in safe Rust, no need to worry.
- A ZST with public constructor can be created by having a semicolon
or an empty pair of brackets or parentheses right after the structure's
name like this: `struct MyStruct;`, `struct MyStruct{}`,
`struct MyStruct()`.
- A ZST with private constructor can be created by having only other ZST
inside it: `struct MyStruct(())` for example.
- I used a ZST here to implement Dependency Injection: I needed a list
of functions to implement (generically), so I used a trait for that.
Only structures or other traits can implement a trait. My implementation,
by using Tokio, didn't have a need to keep any state, so I used a ZST.

## Demo: Directory Listing

![Screenshot](sshot.png "Screenshot")

### Preparation

- A stream with `Item = Result<PathBuf, anyhow::Error>` is defined.
- Tokio has a convenient directory listing procedure that is exposed
as a stream. I wrap Tokio's stream into my stream implementation.

### Procedure

1. The current directory is retrieved.
2. Tokio's stream is constructed and then converted into mine. To do so,
I redirect any errors to `anyhow::Error`, and if there is no error, I
extract its file path and then wrap it in `Ok(_)`.
3. Print each path.

### Reproduction

1. Clone (download) this repository.
2. Run `cargo run`.
3. And then see.
