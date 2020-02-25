# Sled

***EVEN MORE NOT READY FOR PRODUCTION THAN SLED!!!***

An Elixir binding for sled, the champagne of beta embedded databases. There is much work to be done to expose all the functionality in a proper way, but this is a start!

https://github.com/spacejam/sled

  Example

      {:ok, db} = Sled.open("my_db")
      :ok = Sled.insert(db, "hello", "world")
      {:ok, "world"} = Sled.get(db, "hello")

## Installation

If [available in Hex](https://hex.pm/docs/publish), the package can be installed
by adding `sled` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:sled, "~> 0.1.0-alpha"}
  ]
end
```

Documentation can be generated with [ExDoc](https://github.com/elixir-lang/ex_doc)
and published on [HexDocs](https://hexdocs.pm). Once published, the docs can
be found at [https://hexdocs.pm/sled](https://hexdocs.pm/sled).

