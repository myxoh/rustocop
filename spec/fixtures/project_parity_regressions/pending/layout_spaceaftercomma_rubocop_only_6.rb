    set_puppet_conf(confdir, <<-EOF)
      environmentpath=#{environmentpath}
      #{settings.map { |k,v| "#{k}=#{v}" }.join("\n")}
    EOF
    Puppet.initialize_settings
