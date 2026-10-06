===description===
`@taint-sink header $value` raises TaintedHeader.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @taint-sink header $value */
function send_header(string $value): void {}

function test(): void {
    send_header($_GET['v']);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
}
