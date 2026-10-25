===description===
Heredoc indentation and codepoint escape errors use PHP's text, including the expected indentation.
===file===
<?php
function f(): string {
    return <<<'EOT'
      a
//<^^^^^^^ ParseError: Invalid body indentation level (expecting an indentation level of at least 8) [heredoc-body-indentation]
     b
//<^^^^^^ ParseError: Invalid body indentation level (expecting an indentation level of at least 8) [heredoc-body-indentation]
        EOT;
}

function g(): string {
    return "\u{}";
//          ^^^^ ParseError: Invalid UTF-8 codepoint escape sequence [invalid-codepoint-escape]
}
