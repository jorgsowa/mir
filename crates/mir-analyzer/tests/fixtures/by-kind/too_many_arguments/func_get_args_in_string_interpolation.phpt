===description===
func_num_args() inside an interpolated string still lets the function take extra arguments.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f() {
    return "got {${func_num_args()}}";
}
f(1, 2);
