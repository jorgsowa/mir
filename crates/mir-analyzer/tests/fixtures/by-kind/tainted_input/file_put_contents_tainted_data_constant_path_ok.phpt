===description===
writing tainted data to a constant path is not a path-traversal sink (only the path arg matters)
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $data = $_POST['body'];
    file_put_contents('/var/log/app.log', $data);
}
===expect===
