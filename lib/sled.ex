defmodule Sled do
  @moduledoc """
  An Elixir binding for [sled](https://github.com/spacejam/sled), the champagne of beta embedded
  databases.

  A basic example:

      iex> db = Sled.open("test_db")
      iex> Sled.insert(db, "hello", "world")
      iex> Sled.get(db, "hello")
      "world"
  """

  @derive {Inspect, except: [:ref]}
  @enforce_keys [:ref, :path]
  defstruct ref: nil, path: nil

  @typedoc """
  A handle to a sled db.
  """
  @opaque t :: %__MODULE__{ref: reference(), path: binary()}

  @typedoc """
  A handle to a sled tree. Passing a `t:t/0` refers to the "default" tree for the db, while a
  `t:Sled.Tree.t/0` uses the referenced "tenant" tree.
  """
  @type tree :: t() | Sled.Tree.t()

  @doc """
  Open the db with `options`, by default creating it if it doesn't exist.

  If `options` is a path, opens the db at the path with default options, creating it if it
  doesn't exist:

      iex> Sled.open("test_default_db")

  If `options` is a keyword or `Sled.Config.Options` struct, then this function is the same as
  calling `Sled.Config.new/1` and passing the result to `Sled.Config.open/1`.
  """
  @spec open(Path.t() | keyword | Sled.Config.Options.t()) :: t | no_return
  def open(options) when is_binary(options) do
    Sled.Native.sled_open(options)
  end

  def open(options) do
    options
    |> Sled.Config.new()
    |> Sled.Config.open()
  end

  @doc """
  Insert `value` into `db` for `key`.

  Returns `nil` if there was no previous value associated with the key.
  """
  @spec insert(tree, binary, binary) :: binary | nil | no_return
  def insert(db, key, value) do
    Sled.Native.sled_insert(db, key, value)
  end

  @doc """
  Retrieve the value for `key` from `db`.

  Returns `nil` if there is no value associated with the key.
  """
  @spec get(tree, binary) :: binary | nil | no_return
  def get(db, key) do
    Sled.Native.sled_get(db, key)
  end

  @doc """
  Delete the value for `key` from `db`.

  Returns `nil` if there is no value associated with the key.
  """
  @spec remove(tree, binary) :: binary | nil | no_return
  def remove(db, key) do
    Sled.Native.sled_remove(db, key)
  end
end
