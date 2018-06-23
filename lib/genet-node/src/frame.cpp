#include "frame.hpp"
#include "exports.hpp"
#include "layer.hpp"
#include "module.hpp"

namespace genet_node {

void FrameWrapper::init(v8::Local<v8::Object> exports) {
  v8::Local<v8::FunctionTemplate> tpl = Nan::New<v8::FunctionTemplate>(New);
  tpl->InstanceTemplate()->SetInternalFieldCount(1);
  tpl->SetClassName(Nan::New("Frame").ToLocalChecked());

  v8::Local<v8::ObjectTemplate> otl = tpl->InstanceTemplate();
  Nan::SetAccessor(otl, Nan::New("index").ToLocalChecked(), index);
  Nan::SetAccessor(otl, Nan::New("layers").ToLocalChecked(), layers);
  Nan::SetAccessor(otl, Nan::New("treeIndices").ToLocalChecked(), treeIndices);

  v8::Isolate *isolate = v8::Isolate::GetCurrent();
  auto ctor = Nan::GetFunction(tpl).ToLocalChecked();
  auto &cls = Module::current().get(Module::CLASS_FRAME);
  cls.ctor.Reset(isolate, ctor);
}

NAN_METHOD(FrameWrapper::New) {
  if (info.IsConstructCall()) {
    auto frame = info[0];
    if (frame->IsExternal()) {
      auto obj = static_cast<FrameWrapper *>(frame.As<v8::External>()->Value());
      obj->Wrap(info.This());
      info.GetReturnValue().Set(info.This());
    }
  }
}

NAN_GETTER(FrameWrapper::index) {
  FrameWrapper *wrapper = ObjectWrap::Unwrap<FrameWrapper>(info.Holder());
  if (auto frame = wrapper->frame) {
    info.GetReturnValue().Set(genet_frame_index(frame));
  }
}

NAN_GETTER(FrameWrapper::layers) {
  FrameWrapper *wrapper = ObjectWrap::Unwrap<FrameWrapper>(info.Holder());
  if (auto frame = wrapper->frame) {
    uint32_t length = 0;
    auto layers = genet_frame_layers(frame, &length);
    auto array = Nan::New<v8::Array>(length);
    for (uint32_t index = 0; index < length; ++index) {
      array->Set(index, LayerWrapper::wrap(Pointer<Layer>::ref(layers[index])));
    }
    info.GetReturnValue().Set(array);
  }
}

NAN_GETTER(FrameWrapper::treeIndices) {
  FrameWrapper *wrapper = ObjectWrap::Unwrap<FrameWrapper>(info.Holder());
  if (auto frame = wrapper->frame) {
    uint32_t length = 0;
    auto indices = genet_frame_tree_indices(frame, &length);
    auto array = Nan::New<v8::Array>(length);
    for (uint32_t index = 0; index < length; ++index) {
      array->Set(index, Nan::New(static_cast<uint32_t>(indices[index])));
    }
    info.GetReturnValue().Set(array);
  }
}

FrameWrapper::FrameWrapper(const Frame *frame) : frame(frame) {}

FrameWrapper::~FrameWrapper() {}

v8::Local<v8::Object> FrameWrapper::wrap(const Frame *frame) {
  v8::Isolate *isolate = v8::Isolate::GetCurrent();
  const auto &cls = Module::current().get(Module::CLASS_FRAME);
  auto cons = v8::Local<v8::Function>::New(isolate, cls.ctor);
  auto ptr = new FrameWrapper(frame);
  v8::Local<v8::Value> args[] = {Nan::New<v8::External>(ptr)};
  return Nan::NewInstance(cons, 1, args).ToLocalChecked();
}

} // namespace genet_node