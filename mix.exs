defmodule Sled.MixProject do
  use Mix.Project

  @version "0.1.0-alpha.2"

  def project do
    [
      app: :sled,
      version: @version,
      elixir: "~> 1.10",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      compilers: [:rustler] ++ Mix.compilers(),
      rustler_crates: [
        sled_nif: [
          mode: rustc_mode(Mix.env()),
          features: features()
        ]
      ],
      description: description(),
      package: package(),
      source_url: "https://github.com/ericentin/sled",
      docs: [
        main: "Sled",
        extras: ["README.md"],
        source_ref: "v#{@version}"
      ]
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.22.0-rc.0"},
      {:ex_doc, "~> 0.14", only: :dev, runtime: false}
    ]
  end

  defp description do
    "An Elixir binding for sled, the champagne of beta embedded databases."
  end

  @sled_github_url "https://github.com/ericentin/sled"

  defp package do
    [
      files: [
        "lib",
        "native/sled_nif/.cargo",
        "native/sled_nif/src",
        "native/sled_nif/Cargo.toml",
        "native/io_uring_test/src",
        "native/io_uring_test/Cargo.toml",
        ".formatter.exs",
        "mix.exs",
        "README.md",
        "LICENSE*"
      ],
      maintainers: ["Eric Entin"],
      licenses: ["Apache-2.0", "MIT"],
      links: %{
        "GitHub" => @sled_github_url,
        "sled" => "https://github.com/spacejam/sled"
      }
    ]
  end

  defp rustc_mode(:prod), do: :release
  defp rustc_mode(_), do: :debug

  def features do
    if Application.get_env(:sled, :io_uring, false) or io_uring_supported?() do
      ["io_uring"]
    else
      []
    end
  end

  defp io_uring_supported?() do
    is_linux?() and cargo_present?() and io_uring_test_success?()
  end

  defp is_linux? do
    :os.type() == {:unix, :linux}
  end

  defp cargo_present? do
    not is_nil(System.find_executable("cargo"))
  end

  defp io_uring_test_success? do
    case System.cmd("cargo", ["run"],
           stderr_to_stdout: true,
           cd: Path.join([__DIR__, "native", "io_uring_test"])
         ) do
      {_, 0} ->
        true

      {_, 1} ->
        false

      {stdout, exit_status} ->
        io_uring_detection_error(stdout, exit_status)

        false
    end
  end

  defp io_uring_detection_error(stdout, exit_status) do
    Mix.Shell.IO.error([
      """
      Unexpected error determining if io_uring should be enabled.
      Please open an issue at #{@sled_github_url} and include the following log. Thanks!
      stdout:
      """,
      stdout,
      "exited with status: #{exit_status}"
    ])
  end
end
