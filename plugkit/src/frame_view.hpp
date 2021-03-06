#ifndef PLUGKIT_FRAME_VIEW_H
#define PLUGKIT_FRAME_VIEW_H

#include "attr.hpp"
#include "token.hpp"
#include <map>
#include <memory>
#include <vector>

namespace plugkit {

class Frame;
class FrameView;
struct Layer;
struct Attr;

class FrameView final {
public:
  explicit FrameView(Frame *frame);
  ~FrameView();
  const Frame *frame() const;
  const Layer *primaryLayer() const;
  const std::vector<const Layer *> &leafLayers() const;
  const Attr *attr(Token id) const;
  double timestamp() const;
  Slice payload() const;
  void query(Token id, const Layer **layer, const Attr **attr) const;

private:
  FrameView(const FrameView &view) = delete;
  FrameView &operator=(const FrameView &view) = delete;

private:
  const Frame *mFrame;
  std::vector<const Layer *> mLeafLayers;
};
} // namespace plugkit

#endif
