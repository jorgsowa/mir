===description===
Inside isset disabled for dim
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
isset($a[$b]);
===expect===
