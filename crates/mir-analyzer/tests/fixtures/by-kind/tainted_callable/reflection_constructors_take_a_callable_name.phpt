===description===
A client-chosen name lets reflection reach any class or function.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UndefinedClass errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
new ReflectionClass($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
new ReflectionFunction($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
new ReflectionMethod($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
new ReflectionClass(stdClass::class);
