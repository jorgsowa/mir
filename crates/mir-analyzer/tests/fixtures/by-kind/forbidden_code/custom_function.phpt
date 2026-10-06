===description===
Any configured function is forbidden, not only the built-in debug helpers.
===config===
<mir>
  <forbiddenFunctions>
    <function name="eval_like"/>
  </forbiddenFunctions>
</mir>
===file===
<?php
function eval_like(): void {}
function run(): void {
    eval_like();
//  ^^^^^^^^^^^ ForbiddenCode: Use of eval_like is forbidden
    var_dump(1);
}
