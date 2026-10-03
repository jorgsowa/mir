===description===
Date time null first arg
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$date = new DateTime(null);
//                   ^^^^ NullArgument: Argument $datetime of DateTime::__construct() cannot be null
===expect===
