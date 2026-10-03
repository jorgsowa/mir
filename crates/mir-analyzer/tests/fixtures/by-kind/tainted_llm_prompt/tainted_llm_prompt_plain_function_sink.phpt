===description===
@taint-sink on a plain (non-method) function is honored, not just on a
method/static-method call.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @taint-sink llm_prompt $prompt */
function sendPrompt(string $prompt): string {
    return "";
}

sendPrompt((string) $_GET["question"]);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLlmPrompt: Tainted LLM prompt — possible prompt injection
===expect===
