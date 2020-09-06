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
  A reference to a sled db.
  """
  @opaque t :: %__MODULE__{ref: reference(), path: binary()}

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

  defmodule Tree do
    @moduledoc "Defines a struct for sled tenant tree references."

    @derive {Inspect, except: [:ref]}
    @enforce_keys [:ref, :db, :name]
    defstruct ref: nil, db: nil, name: nil

    @typedoc """
    A reference to a sled tenant tree.
    """
    @opaque t :: %__MODULE__{ref: reference(), db: Sled.t(), name: String.t()}
  end

  @doc """
  Open the sled tenant tree in `db` named `name`, creating it if it doesn't exist.
  """
  @spec open_tree(t(), String.t()) :: Sled.Tree.t() | no_return
  def open_tree(db, name) do
    Sled.Native.sled_tree_open(db, name)
  end

  @doc """
  Drop the sled tenant tree in `db` named `name`.

  Returns `true` if a tree was dropped, otherwise `false`.
  """
  @spec drop_tree(t(), String.t()) :: boolean | no_return
  def drop_tree(db, name) do
    Sled.Native.sled_tree_drop(db, name)
  end

  @doc """
  Retrieve the CRC32 of all keys and values in `db`.

  This is O(N) and locks the underlying trees for the duration of the entire scan.
  """
  @spec db_checksum(t()) :: integer | no_return
  def db_checksum(db) do
    Sled.Native.sled_db_checksum(db)
  end

  @doc """
  Retrieve the on-disk size of `db` in bytes.
  """
  @spec size_on_disk(t()) :: integer | no_return
  def size_on_disk(db) do
    Sled.Native.sled_size_on_disk(db)
  end

  @typedoc """
  A reference to a sled tree. Passing a `t:t/0` refers to the "default" tree for the db, while a
  `t:Sled.Tree.t/0` references a "tenant" tree.
  """
  @type tree_ref :: t() | Sled.Tree.t()

  @doc """
  Retrieve the CRC32 of all keys and values in `tree`.

  This is O(N) and locks the underlying tree for the duration of the entire scan.
  """
  @spec checksum(tree_ref) :: integer | no_return
  def checksum(tree) do
    Sled.Native.sled_checksum(tree)
  end

  @doc """
  Synchronously flushes all dirty IO buffers for `tree` and calls fsync.

  If this succeeds, it is guaranteed that all previous writes will be recovered if the system
  crashes. Returns the number of bytes flushed during this call.

  Flushing can take quite a lot of time, and you should measure the performance impact of using it
  on realistic sustained workloads running on realistic hardware.
  """
  @spec flush(tree_ref) :: integer | no_return
  def flush(tree) do
    Sled.Native.sled_flush(tree)
  end

  @doc """
  Insert `value` into `tree` for `key`.

  Returns `nil` if there was no previous value associated with the key.
  """
  @spec insert(tree_ref, binary, binary) :: binary | nil | no_return
  def insert(tree, key, value) do
    Sled.Native.sled_insert(tree, key, value)
  end

  @doc """
  Retrieve the value for `key` from `tree`.

  Returns `nil` if there is no value associated with the key.
  """
  @spec get(tree_ref, binary) :: binary | nil | no_return
  def get(tree, key) do
    Sled.Native.sled_get(tree, key)
  end

  @doc """
  Delete the value for `key` from `tree`.

  Returns `nil` if there is no value associated with the key.
  """
  @spec remove(tree_ref, binary) :: binary | nil | no_return
  def remove(tree, key) do
    Sled.Native.sled_remove(tree, key)
  end
end
