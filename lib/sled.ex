defmodule Sled do
  @moduledoc """
  An Elixir binding for [sled](https://github.com/spacejam/sled), the champagne of beta embedded
  databases.

  A basic example:

      iex> db = Sled.open("test_db")
      iex> Sled.Tree.insert(db, "hello", "world")
      iex> Sled.Tree.get(db, "hello")
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
  Get the tree names saved in `db`.
  """
  @spec tree_names(t()) :: [String.t()] | no_return
  def tree_names(db) do
    Sled.Native.sled_tree_names(db)
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

  @doc """
  Returns true if `db` was recovered from a previous process.
  """
  @spec was_recovered(t()) :: boolean
  def was_recovered(db) do
    Sled.Native.sled_was_recovered(db)
  end

  @doc """
  Generate a monotonic ID from `db`.
  """
  @spec generate_id(t()) :: integer | no_return
  def generate_id(db) do
    Sled.Native.sled_generate_id(db)
  end

  @typedoc """
  Forward-compatible sled export data.
  """
  @type sled_export :: [{binary, binary, [[binary, ...]]}]

  @doc """
  Export all collections in `db`. For use with `import/2` for sled version upgrades.
  """
  @spec export(t()) :: sled_export | no_return
  def export(db) do
    Sled.Native.sled_export(db)
  end

  @doc """
  Import all collections from `export` into `db`. For use with `export/1` for sled version upgrades.
  """
  @spec import(t(), sled_export) :: :ok | no_return
  def import(db, export) do
    Sled.Native.sled_import(db, export)
  end
end
