// use iced::advanced::layout::{Limits, Node};
// use iced::advanced::overlay::Group;
// use iced::advanced::renderer::Style;
// use iced::advanced::widget::tree::{State, Tag};
// use iced::advanced::widget::{Operation, Tree};
// use iced::advanced::{renderer, Clipboard, Layout, Shell, Widget};
// use iced::event::Status;
// use iced::mouse::{Cursor, Interaction};
// use iced::{Element, Event, Length, Point, Rectangle};
//
// struct AbsoluteElement<'a, Message, Renderer> {
//     pub element: Element<'a, Message, Renderer>,
//     pub point: Point,
// }
//
// /// Renders its children at absolute positions using the insert order. Events and mouse interactions are handled in reverse order.
// pub(crate) struct Stack<'a, Message, Renderer> {
//     children: Vec<AbsoluteElement<'a, Message, Renderer>>,
// }
//
// impl<'a, Message, Renderer> Stack<'a, Message, Renderer> {
//     pub fn new() -> Self {
//         Stack {
//             children: Vec::new(),
//         }
//     }
//
//     pub fn with_children(children: Vec<(Element<'a, Message, Renderer>, Point)>) -> Self {
//         Stack {
//             children: children
//                 .into_iter()
//                 .map(|(element, point)| AbsoluteElement { element, point })
//                 .collect(),
//         }
//     }
//
//     pub fn push(mut self, element: Element<'a, Message, Renderer>, point: Point) -> Self {
//         self.children.push(AbsoluteElement { element, point });
//
//         self
//     }
// }
//
// impl<'a, Message, Renderer> Widget<Message, Renderer> for Stack<'a, Message, Renderer>
// where
//     Renderer: renderer::Renderer + 'a,
// {
//     fn width(&self) -> Length {
//         Length::Fill
//     }
//
//     fn height(&self) -> Length {
//         Length::Fill
//     }
//
//     fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
//         let bounds = Node::new(limits.fill());
//
//         let children = self
//             .children
//             .iter()
//             .zip(tree.children.iter_mut())
//             .map(|(child, tree)| {
//                 let mut child_node = child.element.as_widget().layout(tree, renderer, limits);
//                 child_node.move_to(child.point);
//                 child_node
//             })
//             .collect();
//
//         Node::with_children(bounds.size(), children)
//     }
//
//     fn draw(
//         &self,
//         tree: &Tree,
//         renderer: &mut Renderer,
//         theme: &Renderer::Theme,
//         style: &Style,
//         layout: Layout<'_>,
//         cursor: Cursor,
//         viewport: &Rectangle,
//     ) {
//         self.children
//             .iter()
//             .zip(tree.children.iter())
//             .zip(layout.children())
//             .for_each(|((child, tree), layout)| {
//                 renderer.with_layer(layout.bounds(), |renderer| {
//                     child
//                         .element
//                         .as_widget()
//                         .draw(tree, renderer, theme, style, layout, cursor, viewport);
//                 });
//             });
//     }
//
//     fn tag(&self) -> Tag {
//         Tag::stateless()
//     }
//
//     fn state(&self) -> State {
//         State::None
//     }
//
//     fn children(&self) -> Vec<Tree> {
//         self.children
//             .iter()
//             .map(|x| &x.element)
//             .map(Tree::new)
//             .collect()
//     }
//
//     fn diff(&self, tree: &mut Tree) {
//         tree.diff_children(
//             &self
//                 .children
//                 .iter()
//                 .map(|child| child.element.as_widget())
//                 .collect::<Vec<_>>(),
//         );
//     }
//
//     fn operate(
//         &self,
//         tree: &mut Tree,
//         layout: Layout<'_>,
//         renderer: &Renderer,
//         operation: &mut dyn Operation<Message>,
//     ) {
//         tree.children
//             .iter_mut()
//             .zip(self.children.iter())
//             .zip(layout.children())
//             .for_each(|((tree, child), layout)| {
//                 child
//                     .element
//                     .as_widget()
//                     .operate(tree, layout, renderer, operation);
//             });
//     }
//
//     fn on_event(
//         &mut self,
//         tree: &mut Tree,
//         event: Event,
//         layout: Layout<'_>,
//         cursor: Cursor,
//         renderer: &Renderer,
//         clipboard: &mut dyn Clipboard,
//         shell: &mut Shell<'_, Message>,
//         viewport: &Rectangle,
//     ) -> Status {
//         // Give events to the last child who's bounds contain the cursor. Otherwise, ignore the event.
//         // This is done in reverse order so that the top most "focused" child is given the event.
//         for ((child, tree), layout) in self
//             .children
//             .iter_mut()
//             .zip(tree.children.iter_mut())
//             .zip(layout.children().collect::<Vec<Layout>>())
//             .rev()
//         {
//             if cursor.is_over(layout.bounds()) {
//                 return child.element.as_widget_mut().on_event(
//                     tree, event, layout, cursor, renderer, clipboard, shell, viewport,
//                 );
//             }
//         }
//
//         Status::Ignored
//     }
//
//     fn mouse_interaction(
//         &self,
//         tree: &Tree,
//         layout: Layout<'_>,
//         cursor: Cursor,
//         viewport: &Rectangle,
//         renderer: &Renderer,
//     ) -> Interaction {
//         // Give mouse interaction to the last child who's bounds contain the cursor. Otherwise, ignore the event.
//         // This is done in reverse order so that the top most "focused" child is given the cursor.
//         for ((child, tree), layout) in self
//             .children
//             .iter()
//             .zip(tree.children.iter())
//             .zip(layout.children().collect::<Vec<Layout>>())
//             .rev()
//         {
//             if cursor.is_over(layout.bounds()) {
//                 return child
//                     .element
//                     .as_widget()
//                     .mouse_interaction(tree, layout, cursor, viewport, renderer);
//             }
//         }
//
//         Interaction::Idle
//     }
//
//     fn overlay<'b>(
//         &'b mut self,
//         tree: &'b mut Tree,
//         layout: Layout<'_>,
//         renderer: &Renderer,
//     ) -> Option<iced::advanced::overlay::Element<'b, Message, Renderer>> {
//         let children = self
//             .children
//             .iter_mut()
//             .zip(&mut tree.children)
//             .zip(layout.children())
//             .filter_map(|((child, state), layout)| {
//                 child
//                     .element
//                     .as_widget_mut()
//                     .overlay(state, layout, renderer)
//             })
//             .collect::<Vec<_>>();
//
//         (!children.is_empty()).then(|| Group::with_children(children).overlay())
//     }
// }
//
// impl<'a, Message, Renderer> From<Stack<'a, Message, Renderer>> for Element<'a, Message, Renderer>
// where
//     Message: 'a,
//     Renderer: iced::advanced::Renderer + 'a,
// {
//     fn from(row: Stack<'a, Message, Renderer>) -> Self {
//         Self::new(row)
//     }
// }
