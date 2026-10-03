===description===
Array filter third arg will not be used when second null
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
array_filter( $arg, null, ARRAY_FILTER_USE_BOTH );
//            ^^^^ UndefinedVariable: Variable $arg is not defined
===expect===
