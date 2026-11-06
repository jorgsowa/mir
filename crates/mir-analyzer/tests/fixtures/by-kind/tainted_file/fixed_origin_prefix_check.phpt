===description===
`str_starts_with($u, 'https://host/')` fixes the scheme and host as a literal prefix does.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$u = (string) $_GET['u'];
if (str_starts_with($u, 'https://a.com/')) {
    file_get_contents($u);
}
if (str_starts_with($u, 'https://a.com')) {
    file_get_contents($u);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
}
