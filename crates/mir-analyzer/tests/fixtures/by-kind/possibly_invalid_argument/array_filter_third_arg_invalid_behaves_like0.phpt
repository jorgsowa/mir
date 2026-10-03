===description===
Array filter third arg invalid behaves like0
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
array_filter( $arg, "strlen", 3 );
//            ^^^^ UndefinedVariable: Variable $arg is not defined
===expect===
