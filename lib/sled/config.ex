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

    @type maybe(type) :: type | nil

    @type t :: %__MODULE__{
            path: maybe(Path.t()),
            flush_every_ms: maybe(integer()),
            temporary: maybe(boolean()),
            create_new: maybe(boolean()),
            cache_capacity: maybe(integer()),
            print_profile_on_drop: maybe(boolean()),
            use_compression: maybe(boolean()),
            compression_factor: maybe(integer()),
            snapshot_after_ops: maybe(integer()),
            segment_cleanup_threshold: maybe(integer()),
            segment_cleanup_skew: maybe(integer()),
            segment_mode: maybe(atom()),
            snapshot_path: maybe(Path.t()),
            idgen_persist_interval: maybe(integer()),
            read_only: maybe(boolean())
          }
  end

  @opaque t :: %__MODULE__{ref: reference()}

  @doc false
  defstruct ref: nil

  @spec new(Sled.Config.Options.t() | keyword()) :: Sled.Config.t()
  def new(options \\ %Options{})

  def new(%Options{} = options) do
    %__MODULE__{ref: Sled.Native.sled_config_new(options)}
  end

  def new(options) when is_list(options) do
    new(struct(Sled.Config.Options, options))
  end
end
