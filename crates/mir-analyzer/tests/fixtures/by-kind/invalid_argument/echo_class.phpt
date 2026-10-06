===description===
Echo class
===config===
<mir>
  <issueHandlers>
    <ImplicitToStringCast errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
echo (new A);
