===description===
writing tainted data to a constant path is not a path-traversal sink (only the path arg matters)
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $data = $_POST['body'];
    file_put_contents('/var/log/app.log', $data);
}
===expect===
