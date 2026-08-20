module Lockable
  extend ActiveSupport::Concern
end

module RSpec
  module Core

    context "with --out" do
      it "combines with formatters" do
        argv = drb_argv_for(%w[--format h --out report.html])
        expect(argv).to eq(%w[--format h --out report.html])
      end
    end

    context "with -I libs" do
      it "includes multiple paths" do
        argv = drb_argv_for(%w[-I dir_1 -I dir_2 -I dir_3])
        expect(argv).to eq(%w[-I dir_1 -I dir_2 -I dir_3])
      end
    end

    context "with --require" do
      it "includes multiple paths" do
        argv = drb_argv_for(%w[--require dir/ --require file.rb])
        expect(argv).to eq(%w[--require dir/ --require file.rb])
      end
    end
  end
end

ordinary_assignment = "not a call spacing offense"
