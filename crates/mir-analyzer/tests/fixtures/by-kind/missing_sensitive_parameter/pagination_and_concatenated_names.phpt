===description===
All-lowercase names ending in a credential word and one-time passwords are
secrets; pagination tokens aren't.
===config===
<mir>
  <issueHandlers>
    <MissingSensitiveParameter errorLevel="warning"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function connect(string $userpassword, string $otp): void {}
//               ^^^^^^^^^^^^^^^^^^^^ MissingSensitiveParameter: Parameter $userpassword looks like a secret but lacks #[\SensitiveParameter], so its value can appear in stack traces and logs
//                                     ^^^^^^^^^^^ MissingSensitiveParameter: Parameter $otp looks like a secret but lacks #[\SensitiveParameter], so its value can appear in stack traces and logs

function page(string $nextPageToken, ?string $pageToken, string $continuation_token): void {}
