defmodule Sled do
  @moduledoc """
  An Elixir binding for sled, the champagne of beta embedded databases.

  Example

      db = Sled.open!("my_db")
      :ok = Sled.insert!(db, "hello", "world")
      "world" = Sled.get!(db, "hello")
  """

  @enforce_keys [:ref]
  defstruct ref: nil

  @typedoc """
  A handle to a Sled DB.
  """
  @opaque t :: %__MODULE__{ref: reference()}

  defmodule Error do
    defexception [:message]
  end

  @doc """
  Open the db with `options`, by default creating it if it doesn't exist.

  If `options` is a path, opens the db at the path with default options, creating it if it doesn't exist.

  Raises a `Sled.Error` if it fails.
  """
  @spec open!(Path.t() | keyword) :: t
  def open!(options) when is_binary(options) do
    Sled.Native.sled_open(options)
  end

  def open!(options) when is_list(options) do
    options
    |> Sled.Config.new!()
    |> Sled.Config.open!()
  end

  @doc """
  Insert into `db` key `k` and value `v`.

  Raises a `Sled.Error` if it fails.
  """
  @spec insert!(t, binary, binary) :: :ok
  def insert!(db, k, v) do
    Sled.Native.sled_insert(db, k, v)
  end

  @doc """
  From `db`, retrieve the value for key `k`.

  Returns `nil` if there is no value associated with the key.

  Raises a `Sled.Error` if it fails.
  """
  @spec get!(t, binary) :: binary | nil
  def get!(db, k) do
    Sled.Native.sled_get(db, k)
  end
end
