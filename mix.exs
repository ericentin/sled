defmodule Sled.MixProject do
  use Mix.Project

  @version "0.1.0-alpha.1"

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
    "An Elixir binding for Sled, the champagne of beta embedded databases."
  end

  @sled_github_url "https://github.com/ericentin/sled"

  defp package do
    [
      files: [
        "lib",
        "native/sled_nif/.cargo",
        "native/sled_nif/src",
        "native/sled_nif/Cargo.toml",
        ".formatter.exs",
        "mix.exs",
        "README.md"
      ],
      maintainers: ["Eric Entin"],
      licenses: ["Apache-2.0", "MIT"],
      links: %{
        "GitHub" => @sled_github_url,
        "Sled" => "https://github.com/spacejam/sled"
      }
    ]
  end

  defp rustc_mode(:prod), do: :release
  defp rustc_mode(_), do: :debug

  features =
    try do
      # If not linux, big nope
      if :os.type() != {:unix, :linux}, do: throw([])

      # If cargo isn't installed, let rustler handle it, we'll recalculate features later
      if !(cargo = System.find_executable("cargo")), do: throw([])

      native_path = Path.join(__DIR__, "native")
      rio_nop_root_path = Path.join(native_path, "rio_nop")
      rio_nop_path = Path.join([rio_nop_root_path, "bin", "nop"])

      # Install rio nop if needed
      unless File.exists?(rio_nop_path) do
        case System.cmd(
               cargo,
               ~w[install rio --example nop --root #{rio_nop_root_path}],
               stderr_to_stdout: true
             ) do
          {_, 0} ->
            :ok

          {stdout, exit_status} ->
            Mix.Shell.IO.error([
              """
              Unexpected error determining if io_uring should be enabled.
              Please open an issue at #{@sled_github_url} and include the following log. Thanks!
              stdout:
              """,
              stdout,
              "exited with status: #{exit_status}"
            ])

            throw([])
        end
      end

      case System.cmd(rio_nop_path, [], stderr_to_stdout: true) do
        {_stdout, 0} ->
          ["io_uring"]

        {_, 101} ->
          []

        {stdout, exit_status} ->
          Mix.Shell.IO.error([
            """
            Unexpected error determining if io_uring should be enabled.
            Please open an issue at #{@sled_github_url} and include the following log. Thanks!
            stdout:
            """,
            stdout,
            "exited with status: #{exit_status}"
          ])

          []
      end
    catch
      [] -> []
    end

  def features do
    if Application.get_env(:sled, :io_uring) do
      ["io_uring"]
    else
      unquote(features)
    end
  end
end
