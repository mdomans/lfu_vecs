# lfu_vecs
LFU implemented in very basic (no interior mutability) Rust.

### Why and how?

Not so long ago a paper on O(1) LFU design (http://dhruvbird.com/lfu.pdf) was pretty popular in caching world. While LFU isn't (on it's own) that good of a general cache algorithm, people do like bragging about implementing a O(1) algorithm :)

The paper outlines the algorithm using doubly linked list. That's both interesting as a problem for learning Rust (because doubly linked lists are deemed hard) syntax and a counter example of how to implement something in Rust. I came to that conclusion comparing version with doubly linked lists that requires use of interior mutability pattern in Rust. That in turn means runtime vs. compile time checking and generally makes for worse performance overall due to simple problems such as poor compiler-level optimizations.

So as an exercise in writing idiomatic performant Rust that's easy to read, argue about and still correct I added this version.

The design principles are fairly simple:
 
* store data about keys and how frequent they are using `FrequencyNode` struct
* store said structs in a `Vec<FrequencyNode>`
* at the moment frequency simply maps to index in that list 
* values for keys are stored in a `HashMap` using `Item` struct
* `Item` itself simply contains the current index of `FrequencyNode` storing the key and `Bytes` field for data

Assuming Rust's `HashMap` `get`, `insert` and `remove` operations are O(1) this boils down to question of what's the added complexity coming from storing frequency data in Vec<T>:
  
  * `get` means we need to grab the `Item` from `HashMap`, pull key from `Vec<FrequencyNode>`, assign it to next `Vec<FrequencyNode>` and incrment it's `index`
  * `set` is simply insert into `HashMap` and add key to `Vec<FrequencyNode>` at index 0
  * only worrying point at the moment is adding new `FrequencyNode>` since that's O(m) where m is `Vec.len()`
