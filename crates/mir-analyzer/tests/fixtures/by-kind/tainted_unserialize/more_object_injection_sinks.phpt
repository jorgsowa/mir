===description===
Other deserializers can instantiate classes just like `unserialize`.
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
igbinary_unserialize($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function igbinary_unserialize() is not defined
yaml_parse($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
session_decode($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
igbinary_unserialize('a');
//<^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function igbinary_unserialize() is not defined
