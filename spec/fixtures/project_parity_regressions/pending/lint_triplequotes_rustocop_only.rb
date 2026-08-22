    let(:content) { "(make-pathname :defaults name\n:type \"assem\")" }
    let(:multiline_content) do
      %q(
      def test(input):
        """This is line 1 of a multi-line comment.
        This is line 2.
        """
      )
    end
