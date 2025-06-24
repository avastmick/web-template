# Development servers
# Default: hot reload and logging are ENABLED
# To disable hot reload: OVERMIND_HOTRELOAD=false just dev
# To disable logging: OVERMIND_LOGGING=false just dev
Server: bash -c 'just server-dev ${OVERMIND_HOTRELOAD:+--hotreload} ${OVERMIND_LOGGING:+--log}'
Client: bash -c 'just client-dev ${OVERMIND_LOGGING:+--log}'
