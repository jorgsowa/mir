===description===
FirstClassCallable:UndefinedStaticMethod
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Widget {}
$closure = Widget::undefined(...);
//                 ^^^^^^^^^ UndefinedMethod: Method Widget::undefined() does not exist
$count = $closure();
