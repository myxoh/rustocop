      allow(Open3).to receive(:popen2e).with({}, *%w[
        kind get clusters
      ]).and_return([clusters, command_status])
