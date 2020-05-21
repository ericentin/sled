defmodule Sled.Config do
  defmodule Options do
    defstruct path: nil,
              flush_every_ms: nil,
              temporary: nil,
              create_new: nil,
              cache_capacity: nil,
              print_profile_on_drop: nil,
              use_compression: nil,
              compression_factor: nil,
              snapshot_after_ops: nil,
              segment_cleanup_threshold: nil,
              segment_cleanup_skew: nil,
              segment_mode: nil,
              snapshot_path: nil,
              idgen_persist_interval: nil,
              read_only: nil

    @typedoc """
    Sled configuration parameters.

    For more info, refer to https://docs.rs/sled/0.31/sled/struct.Config.html#methods.
    """
    @type t :: %__MODULE__{
            path: Path.t() | nil,
            flush_every_ms: integer() | false | nil,
            temporary: boolean() | nil,
            create_new: boolean() | nil,
            cache_capacity: integer() | nil,
            print_profile_on_drop: boolean() | nil,
            use_compression: boolean() | nil,
            compression_factor: integer() | nil,
            snapshot_after_ops: integer() | nil,
            segment_cleanup_threshold: integer() | nil,
            segment_cleanup_skew: integer() | nil,
            segment_mode: :gc | :linear | nil,
            snapshot_path: Path.t() | false | nil,
            idgen_persist_interval: integer() | nil,
            read_only: boolean() | nil
          }
  end

  @enforce_keys [:ref]
  defstruct ref: nil

  @typedoc """
  A handle to a cached Sled config.
  """
  @opaque t :: %__MODULE__{ref: reference()}

  @doc """
  Create a Sled config that you can use with `open/1`.

  Raises a `Sled.Error` if s
  """
  @spec new!(keyword | Options.t()) :: t
  def new!(options \\ %Options{})

  def new!(options) when is_list(options) do
    new!(struct!(Sled.Config.Options, options))
  end

  def new!(options) do
    Sled.Native.sled_config_new(options)
  end

  @spec open!(t) :: Sled.t()
  def open!(config) do
    Sled.Native.sled_config_open(config)
  end

  parent = __MODULE__

  defimpl Inspect do
    def inspect(%unquote(parent){} = config, _opts) do
      "#Sled.Config<sled::" <> Sled.Native.sled_config_inspect(config) <> ">"
    end
  end
end
