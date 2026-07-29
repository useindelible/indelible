export function nodePath(node: Node): string {
  const parts: number[] = []
  const documentElement = node.ownerDocument?.documentElement
  let current: Node | null = node

  while (current && current !== documentElement) {
    const parent: Node | null = current.parentNode
    if (!parent) break
    parts.unshift(Array.from(parent.childNodes).indexOf(current as ChildNode))
    current = parent
  }

  return parts.join('/')
}
