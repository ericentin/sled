defmodule Sled.Config do
  @moduledoc "Configuration for sled."

  defmodule Options do
    @moduledoc "Defines a struct for `Sled.Config` options."

    defstruct path: nil,
              cache_capacity: nil,
              mode: nil,
              use_compression: nil,
              compression_factor: nil,
              temporary: nil,
              create_new: nil,
              print_profile_on_drop: nil

    @typedoc """
    sled configuration options.

    For more info, refer to https://docs.rs/sled/0.31/sled/struct.Config.html#methods.
    """
    @type t :: %__MODULE__{
            path: Path.t() | nil,
            cache_capacity: integer() | nil,
            mode: :low_space | :high_throughput | nil,
            use_compression: boolean() | nil,
            compression_factor: integer() | nil,
            temporary: boolean() | nil,
            create_new: boolean() | nil,
            print_profile_on_drop: boolean() | nil
          }
  end

  @enforce_keys [:ref]
  defstruct ref: nil

  @typedoc """
  A handle to a cached sled config.
  """
  @opaque t :: %__MODULE__{ref: reference()}

  @doc """
  Create a sled config for `options`.

  You can pass keyword arguments:

      iex> Sled.Config.new(path: "test_keyword_config_db")

  or, you can use the `Sled.Config.Options` struct, if you prefer:

      iex> Sled.Config.new(%Sled.Config.Options{path: "test_struct_config_db"})
  """
  @spec new(keyword | Options.t()) :: t | no_return
  def new(options \\ %Options{})

  def new(options) when is_list(options) do
    new(struct!(Sled.Config.Options, options))
  end

  def new(%Sled.Config.Options{} = options) do
    Sled.Native.sled_config_new(options)
  end

  @doc """
  Open the sled database for the given `config`.

      iex> config = Sled.Config.new(path: "test_config_db")
      iex> Sled.Config.open(config)
      #Sled<path: "test_config_db">
  """
  @spec open(t) :: Sled.t() | no_return
  def open(config) do
    Sled.Native.sled_config_open(config)
  end

  parent = __MODULE__

  defimpl Inspect do
    @impl true
    def inspect(%unquote(parent){} = config, _opts) do
      "##{unquote(inspect(parent))}<sled::" <> Sled.Native.sled_config_inspect(config) <> ">"
    end
  end
end
