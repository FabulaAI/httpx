import typing

PrimitiveData = typing.Optional[typing.Union[str, int, float, bool]]
QueryParamTypes = typing.Union[
    "QueryParams",
    typing.Mapping[str, typing.Union[PrimitiveData, typing.Sequence[PrimitiveData]]],
    typing.List[typing.Tuple[str, PrimitiveData]],
    typing.Tuple[typing.Tuple[str, PrimitiveData], ...],
    str,
    bytes,
]

@typing.final
class QueryParams(typing.Mapping[str, str]):
    def __new__(
        cls, *args: QueryParamTypes | None, **kwargs: typing.Any
    ) -> QueryParams: ...
    def keys(self) -> typing.KeysView[str]:
        """
        Return all the keys in the query params.

        Usage:

        ```
        q = httpx.QueryParams("a=123&a=456&b=789")
        assert list(q.keys()) == ["a", "b"]
        ```
        """

    def values(self) -> typing.ValuesView[str]:
        """
        Return all the values in the query params. If a key occurs more than once
        only the first item for that key is returned.

        Usage:

        ```
        q = httpx.QueryParams("a=123&a=456&b=789")
        assert list(q.values()) == ["123", "789"]
        ```
        """

    def items(self) -> typing.ItemsView[str, str]:
        """
        Return all items in the query params. If a key occurs more than once
        only the first item for that key is returned.

        Usage:

        q = httpx.QueryParams("a=123&a=456&b=789")
        assert list(q.items()) == [("a", "123"), ("b", "789")]
        """

    def multi_items(self) -> list[tuple[str, str]]:
        """
        Return all items in the query params. Allow duplicate keys to occur.

        Usage:

        ```
        q = httpx.QueryParams("a=123&a=456&b=789")
        assert list(q.multi_items()) == [("a", "123"), ("a", "456"), ("b", "789")]
        ```
        """

    def get(self, key: typing.Any, default: typing.Any = None) -> typing.Any:
        """
        Get a value from the query param for a given key. If the key occurs
        more than once, then only the first value is returned.

        Usage:

        ```
        q = httpx.QueryParams("a=123&a=456&b=789")
        assert q.get("a") == "123"
        ```
        """

    def get_list(self, key: str) -> list[str]:
        """
        Get all values from the query param for a given key.

        Usage:

        ```
        q = httpx.QueryParams("a=123&a=456&b=789")
        assert q.get_list("a") == ["123", "456"]
        ```
        """

    def set(self, key: str, value: typing.Any = None) -> QueryParams:
        """
        Return a new QueryParams instance, setting the value of a key.

        Usage:

        ```
        q = httpx.QueryParams("a=123")
        q = q.set("a", "456")
        assert q == httpx.QueryParams("a=456")
        ```
        """

    def add(self, key: str, value: typing.Any = None) -> QueryParams:
        """
        Return a new QueryParams instance, setting or appending the value of a key.

        Usage:

        ```
        q = httpx.QueryParams("a=123")
        q = q.add("a", "456")
        assert q == httpx.QueryParams("a=123&a=456")
        ```
        """

    def remove(self, key: str) -> QueryParams:
        """
        Return a new QueryParams instance, removing the value of a key.

        Usage:
        ```
        q = httpx.QueryParams("a=123")
        q = q.remove("a")
        assert q == httpx.QueryParams("")
        ```
        """

    def merge(self, params: QueryParamTypes | None = None) -> QueryParams:
        """
        Return a new QueryParams instance, updated with.

        Usage:
        ```
        q = httpx.QueryParams("a=123")
        q = q.merge({"b": "456"})
        assert q == httpx.QueryParams("a=123&b=456")

        q = httpx.QueryParams("a=123")
        q = q.merge({"a": "456", "b": "789"})
        assert q == httpx.QueryParams("a=456&b=789")
        ```
        """

    def __getitem__(self, key: typing.Any) -> str: ...
    def __contains__(self, key: typing.Any) -> bool: ...
    def __iter__(self) -> typing.Iterator[typing.Any]: ...
    def __len__(self) -> int: ...
    def __bool__(self) -> bool: ...
    def __hash__(self) -> int: ...
    def __eq__(self, other: typing.Any) -> bool: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def update(self, params: QueryParamTypes | None = None) -> None: ...
    def __setitem__(self, key: str, value: str) -> None: ...

def normalize_path(path: str) -> str:
    """
    Drop "." and ".." segments from a URL path.

    For example:

        normalize_path("/path/./to/somewhere/..") == "/path/to"
    """

def quote(string: str, safe: str) -> str:
    """
    Use percent-encoding to quote a string, omitting existing '%xx' escape sequences.

    See: https://www.rfc-editor.org/rfc/rfc3986#section-2.1

    * `string`: The string to be percent-escaped.
    * `safe`: A string containing characters that may be treated as safe, and do not
        need to be escaped. Unreserved characters are always treated as safe.
        See: https://www.rfc-editor.org/rfc/rfc3986#section-2.3
    """

def unquote(value: str) -> str: ...
def find_ascii_non_printable(s: str) -> typing.Optional[int]: ...
def validate_path(path: str, has_scheme: bool, has_authority: bool) -> None:
    """
    Path validation rules that depend on if the URL contains
    a scheme or authority component.

    See https://datatracker.ietf.org/doc/html/rfc3986.html#section-3.3

    ---

    If a URI contains an authority component, then the path component
    must either be empty or begin with a slash ("/") character."

    ---

    If a URI does not contain an authority component, then the path cannot begin
    with two slash characters ("//").

    ---

    In addition, a URI reference (Section 4.1) may be a relative-path reference,
    in which case the first path segment cannot contain a colon (":") character.
    """

def normalize_port(port: int | str | None, scheme: str) -> int | None:
    """
    From https://tools.ietf.org/html/rfc3986#section-3.2.3

    "A scheme may define a default port.  For example, the "http" scheme
    defines a default port of "80", corresponding to its reserved TCP
    port number.  The type of port designated by the port number (e.g.,
    TCP, UDP, SCTP) is defined by the URI scheme.  URI producers and
    normalizers should omit the port component and its ":" delimiter if
    port is empty or if its value would be the same as that of the
    scheme's default."

    See https://url.spec.whatwg.org/#url-miscellaneous
    """

class InvalidURL(Exception):
    def __init__(self, message: str) -> None: ...

class CookieConflict(Exception):
    """
    Attempted to lookup a cookie by name, but multiple cookies existed.

    Can occur when calling `response.cookies.get(...)`.
    """

    def __init__(self, message: str) -> None: ...

def encode_host(host: str) -> str: ...
