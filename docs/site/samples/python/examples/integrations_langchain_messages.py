"""Use Prompting Press prompts with LangChain / LangGraph (Python).

Prompting Press renders a composition to an ordered list of role-tagged
messages. LangChain accepts plain ``{"role", "content"}`` dicts directly, so the
bridge is a one-line key rename (``text`` -> ``content``): map each rendered
message and hand the list straight to a chat model or a LangGraph node.

Note: do NOT route already-rendered text through
``ChatPromptTemplate.from_messages`` with the tuple/dict shorthand. That path
treats ``content`` as an f-string template and raises on any literal ``{...}``
(e.g. JSON) in your rendered text. Prompting Press already did the templating —
feed the model/graph the messages directly. Standalone.
"""

from langchain_core.language_models.fake_chat_models import FakeListChatModel
from langchain_core.messages import AIMessage, HumanMessage, SystemMessage
from langchain_core.messages.utils import convert_to_messages
from prompting_press import Composition, Prompt
from pydantic import BaseModel


class SysVars(BaseModel):
    instructions: str


class UserVars(BaseModel):
    payload: str


def to_langchain(messages):
    """Map a Prompting Press composition result to LangChain message dicts.

    ``[{role, text}]`` -> ``[{"role", "content"}]``. Order and role are
    preserved; ``role`` values (system/user/assistant) are accepted by
    LangChain as-is.
    """
    return [{"role": m.role, "content": m.text} for m in messages]


# Optional: if you want typed message OBJECTS instead of dicts, map role -> class.
_ROLE_TO_MESSAGE = {
    "system": SystemMessage,
    "user": HumanMessage,
    "assistant": AIMessage,
}


def to_langchain_objects(messages):
    return [_ROLE_TO_MESSAGE[m.role](content=m.text) for m in messages]


# Build a composition. The user turn deliberately contains literal braces to
# prove rendered text is NOT re-templated by the direct path.
sys_prompt = Prompt(
    {
        "name": "system-preamble",
        "role": "system",
        "body": "{{ instructions }}",
        "variables": {"instructions": {"type": "string", "trusted": True}},
    }
)
user_prompt = Prompt(
    {
        "name": "user-turn",
        "role": "user",
        "body": "{{ payload }}",
        "variables": {"payload": {"type": "string", "trusted": False}},
    }
)

comp = Composition()
comp.append(sys_prompt, SysVars(instructions="You are a helpful assistant."))
comp.append(user_prompt, UserVars(payload='Return this exactly: {"k": 1}'))

lc_messages = to_langchain(comp.resolve())

# Hand the messages straight to a chat model (FakeListChatModel stands in for a
# real one so this runs offline — a real app uses ChatOpenAI, ChatBedrock, ...).
model = FakeListChatModel(responses=["ok"])
reply = model.invoke(lc_messages)
print(reply.content)  # "ok"

# --- assertions (this file is executed by CI) ---

# Key rename only: order + role preserved, content == text verbatim.
assert lc_messages == [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": 'Return this exactly: {"k": 1}'},
]

# LangChain coerces the dicts to the right message classes, and the literal
# braces survive (they would raise under ChatPromptTemplate's template path).
coerced = convert_to_messages(lc_messages)
assert [type(m).__name__ for m in coerced] == ["SystemMessage", "HumanMessage"]
assert coerced[1].content == 'Return this exactly: {"k": 1}'

# The typed-object variant maps to the same classes.
objs = to_langchain_objects(comp.resolve())
assert [type(m).__name__ for m in objs] == ["SystemMessage", "HumanMessage"]
assert isinstance(reply, AIMessage)
