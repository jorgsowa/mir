===description===
FirstClassCallable:UndefinedStaticMethod
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class Widget {}
$closure = Widget::undefined(...);
$count = $closure();
===expect===
UndefinedMethod@3:19-3:28: Method Widget::undefined() does not exist
